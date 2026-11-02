package app.devroom.mobile;

import android.Manifest;
import android.app.PendingIntent;
import android.content.Context;
import android.content.Intent;
import android.net.Uri;
import android.os.Build;
import android.view.View;
import android.view.ViewGroup;
import android.webkit.WebView;
import android.widget.FrameLayout;
import androidx.activity.OnBackPressedCallback;
import androidx.core.graphics.Insets;
import androidx.core.view.ViewCompat;
import androidx.core.view.WindowInsetsCompat;
import androidx.core.content.ContextCompat;
import androidx.core.content.pm.PackageInfoCompat;
import com.getcapacitor.JSObject;
import com.getcapacitor.PermissionState;
import com.getcapacitor.Plugin;
import com.getcapacitor.PluginCall;
import com.getcapacitor.PluginMethod;
import com.getcapacitor.annotation.CapacitorPlugin;
import com.getcapacitor.annotation.Permission;
import com.getcapacitor.annotation.PermissionCallback;

/** Bridge for the web shell: background watching, opening links outside the app, app version. */
@CapacitorPlugin(
    name = "DevRoom",
    permissions = { @Permission(alias = "notifications", strings = { Manifest.permission.POST_NOTIFICATIONS }) }
)
public class DevRoomPlugin extends Plugin {

    @PluginMethod
    public void startWatch(PluginCall call) {
        String url = call.getString("url");
        if (url == null || url.isEmpty()) { call.reject("url is required"); return; }
        getContext().getSharedPreferences(WatchService.PREFS, Context.MODE_PRIVATE).edit()
            .putString("url", url).putString("token", call.getString("token", "")).putString("alt", call.getString("alt", "")).apply();
        if (Build.VERSION.SDK_INT >= 33 && getPermissionState("notifications") != PermissionState.GRANTED) {
            requestPermissionForAlias("notifications", call, "afterPermission");
            return;
        }
        launch();
        JSObject r = new JSObject();
        r.put("granted", true);
        call.resolve(r);
    }

    @PermissionCallback
    private void afterPermission(PluginCall call) {
        launch();
        JSObject r = new JSObject();
        r.put("granted", getPermissionState("notifications") == PermissionState.GRANTED);
        call.resolve(r);
    }

    private void launch() {
        ContextCompat.startForegroundService(getContext(), new Intent(getContext(), WatchService.class));
    }

    @PluginMethod
    public void stopWatch(PluginCall call) {
        getContext().stopService(new Intent(getContext(), WatchService.class));
        call.resolve();
    }

    @PluginMethod
    public void openExternal(PluginCall call) {
        String url = call.getString("url");
        if (url == null) { call.reject("url is required"); return; }
        Intent i = new Intent(Intent.ACTION_VIEW, Uri.parse(url));
        i.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
        getContext().startActivity(i);
        call.resolve();
    }

    /** Opens the official Claude Code chat inside the app. */
    @PluginMethod
    public void openClaude(PluginCall call) {
        String url = call.getString("url", "https://claude.ai/code");
        Intent i = new Intent(getContext(), ClaudeActivity.class);
        i.putExtra("url", url);
        getActivity().startActivity(i);
        call.resolve();
    }

    // ---- The real Claude Code chat on the office's Chat tab ----
    private WebView claude;                   // kept alive while hidden, so the chat and sign-in stay put
    private OnBackPressedCallback back;
    private int[] want = { 0, 0, 0, 0 };      // requested spot in the shell page, in CSS px
    private int ime = 0;                      // keyboard height in px

    /** Shows the Claude view over a spot of the shell page ({show, x, y, w, h} in CSS px), or hides it. */
    @PluginMethod
    public void claudeView(PluginCall call) {
        boolean show = call.getBoolean("show", false);
        int[] r = { call.getInt("x", 0), call.getInt("y", 0), call.getInt("w", 0), call.getInt("h", 0) };
        getActivity().runOnUiThread(() -> {
            if (!show) {
                if (claude != null) claude.setVisibility(View.GONE);
                if (back != null) back.setEnabled(false);
                call.resolve();
                return;
            }
            if (claude == null) {
                claude = ClaudeWeb.create(getActivity());
                claude.setVisibility(View.GONE);
                ViewGroup root = getActivity().findViewById(android.R.id.content);
                root.addView(claude, new FrameLayout.LayoutParams(1, 1));
                claude.loadUrl(ClaudeWeb.HOME);
                // the keyboard must never cover the message box
                ViewCompat.setOnApplyWindowInsetsListener(root, (v, in) -> {
                    Insets k = in.getInsets(WindowInsetsCompat.Type.ime());
                    if (k.bottom != ime) { ime = k.bottom; place(); }
                    return in;
                });
                back = new OnBackPressedCallback(false) {
                    @Override public void handleOnBackPressed() {
                        if (claude != null && claude.getVisibility() == View.VISIBLE && claude.canGoBack()) { claude.goBack(); return; }
                        setEnabled(false);
                        getActivity().getOnBackPressedDispatcher().onBackPressed();
                        setEnabled(claude != null && claude.getVisibility() == View.VISIBLE);
                    }
                };
                getActivity().getOnBackPressedDispatcher().addCallback(getActivity(), back);
            }
            want = r;
            place();
            claude.setVisibility(View.VISIBLE);
            back.setEnabled(true);
            call.resolve();
        });
    }

    /** Lays the Claude view exactly over the requested spot of the shell page, above the keyboard. */
    private void place() {
        if (claude == null) return;
        float d = getActivity().getResources().getDisplayMetrics().density;
        ViewGroup root = getActivity().findViewById(android.R.id.content);
        int[] a = new int[2], b = new int[2];
        root.getLocationInWindow(a);
        getBridge().getWebView().getLocationInWindow(b);
        int x = Math.round(want[0] * d) + b[0] - a[0], y = Math.round(want[1] * d) + b[1] - a[1];
        int w = Math.max(1, Math.round(want[2] * d)), h = Math.round(want[3] * d);
        if (ime > 0) h = Math.min(h, root.getHeight() - ime - y);
        FrameLayout.LayoutParams lp = new FrameLayout.LayoutParams(w, Math.max(1, h));
        lp.leftMargin = x;
        lp.topMargin = y;
        claude.setLayoutParams(lp);
    }

    /** The APK is kept here between the download and the install. */
    private java.io.File apkFile() { return new java.io.File(new java.io.File(getContext().getCacheDir(), "updates"), "DevRoom.apk"); }

    private static boolean okUrl(String url) { return url != null && url.startsWith("https://github.com/Ronanski/dev-room-app/releases/"); }

    /** Downloads the APK (progress as "updateProgress" events). Throws on failure. */
    private void download(String url) throws Exception {
        java.io.File apk = apkFile();
        apk.getParentFile().mkdirs();
        java.io.File part = new java.io.File(apk.getParentFile(), "DevRoom.apk.part");
        java.net.HttpURLConnection c = (java.net.HttpURLConnection) new java.net.URL(url).openConnection();
        c.setInstanceFollowRedirects(true);
        c.setConnectTimeout(15000);
        c.setReadTimeout(30000);
        int code = c.getResponseCode();
        if (code != 200) throw new Exception("HTTP " + code);
        long total = c.getContentLengthLong(), got = 0;
        int last = -1;
        try (java.io.InputStream in = c.getInputStream(); java.io.OutputStream out = new java.io.FileOutputStream(part)) {
            byte[] buf = new byte[64 * 1024];
            int n;
            while ((n = in.read(buf)) > 0) {
                out.write(buf, 0, n);
                got += n;
                int pct = total > 0 ? (int) (got * 100 / total) : -1;
                if (pct != last) { last = pct; JSObject p = new JSObject(); p.put("pct", pct); notifyListeners("updateProgress", p); }
            }
        } finally { c.disconnect(); }
        if (part.length() < 1024 * 1024) throw new Exception("download looks wrong");
        apk.delete();
        if (!part.renameTo(apk)) throw new Exception("cannot keep the download");
    }

    /** Android's "install unknown apps" switch for Dev Room; opens that setting when it is off. */
    private boolean canInstall() {
        Context ctx = getContext();
        if (Build.VERSION.SDK_INT >= 26 && !ctx.getPackageManager().canRequestPackageInstalls()) {
            Intent s = new Intent(android.provider.Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES, Uri.parse("package:" + ctx.getPackageName()));
            s.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
            try { ctx.startActivity(s); } catch (Exception ignored) { }
            return false;
        }
        return true;
    }

    /**
     * Installs the downloaded APK through an install session. On Android 12+ a self-update needs no tap
     * (USER_ACTION_NOT_REQUIRED); when Android still asks, UpdateReceiver opens the confirmation. The app is
     * replaced right after, so this is called when you leave the app or after the countdown.
     */
    private void sessionInstall(java.io.File apk) throws Exception {
        Context ctx = getContext();
        android.content.pm.PackageInstaller pi = ctx.getPackageManager().getPackageInstaller();
        android.content.pm.PackageInstaller.SessionParams sp = new android.content.pm.PackageInstaller.SessionParams(android.content.pm.PackageInstaller.SessionParams.MODE_FULL_INSTALL);
        sp.setAppPackageName(ctx.getPackageName());
        sp.setSize(apk.length());
        if (Build.VERSION.SDK_INT >= 31) sp.setRequireUserAction(android.content.pm.PackageInstaller.SessionParams.USER_ACTION_NOT_REQUIRED);
        if (Build.VERSION.SDK_INT >= 34) sp.setRequestUpdateOwnership(true);
        int id = pi.createSession(sp);
        try (android.content.pm.PackageInstaller.Session s = pi.openSession(id)) {
            try (java.io.InputStream in = new java.io.FileInputStream(apk); java.io.OutputStream out = s.openWrite("DevRoom.apk", 0, apk.length())) {
                byte[] buf = new byte[64 * 1024];
                int n;
                while ((n = in.read(buf)) > 0) out.write(buf, 0, n);
                s.fsync(out);
            }
            Intent cb = new Intent(ctx, UpdateReceiver.class).setAction(UpdateReceiver.ACTION_RESULT);
            int flags = PendingIntent.FLAG_UPDATE_CURRENT | (Build.VERSION.SDK_INT >= 31 ? PendingIntent.FLAG_MUTABLE : 0);
            s.commit(PendingIntent.getBroadcast(ctx, id, cb, flags).getIntentSender());
        }
    }

    /** Downloads the update now, to install it later (when you leave the app). Resolves { ok } or { needsPermission }. */
    @PluginMethod
    public void downloadUpdate(PluginCall call) {
        String url = call.getString("url");
        if (!okUrl(url)) { call.reject("bad url"); return; }
        new Thread(() -> {
            try { download(url); JSObject r = new JSObject(); r.put("ok", true); call.resolve(r); }
            catch (Exception e) { call.reject("Hindi na-download ang update: " + e.getMessage()); }
        }).start();
    }

    /**
     * Installs the update: the APK downloaded before (or downloads it now). With silent:true it uses the install
     * session (no tap on Android 12+); otherwise Android's installer screen opens, as before.
     * The first time, Android asks to allow installs from Dev Room; the call then resolves with needsPermission.
     */
    @PluginMethod
    public void installUpdate(PluginCall call) {
        String url = call.getString("url");
        boolean silent = Boolean.TRUE.equals(call.getBoolean("silent", false));
        if (!okUrl(url)) { call.reject("bad url"); return; }
        if (!canInstall()) { JSObject r = new JSObject(); r.put("needsPermission", true); call.resolve(r); return; }
        Context ctx = getContext();
        new Thread(() -> {
            java.io.File apk = apkFile();
            try {
                if (!silent || !apk.isFile() || apk.length() < 1024 * 1024) download(url);
                if (silent) {
                    sessionInstall(apk);
                } else {
                    Uri uri = androidx.core.content.FileProvider.getUriForFile(ctx, ctx.getPackageName() + ".fileprovider", apk);
                    Intent i = new Intent(Intent.ACTION_VIEW);
                    i.setDataAndType(uri, "application/vnd.android.package-archive");
                    i.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_ACTIVITY_NEW_TASK);
                    ctx.startActivity(i);
                }
                JSObject r = new JSObject(); r.put("ok", true); r.put("silent", silent); call.resolve(r);
            } catch (Exception e) {
                call.reject("Hindi na-install ang update: " + e.getMessage());
            }
        }).start();
    }

    /**
     * How much of the web view the status bar, the navigation bar (keys or gesture bar) and a notch cover, in CSS
     * pixels. The office runs in an iframe, where env(safe-area-inset-*) is always 0, so the shell passes these on.
     * Measured from where the web view really is, so it is 0 when the app is not drawn under the bars.
     */
    @PluginMethod
    public void insets(PluginCall call) {
        getActivity().runOnUiThread(() -> {
            JSObject r = new JSObject();
            try {
                View wv = getBridge().getWebView();
                View decor = getActivity().getWindow().getDecorView();
                WindowInsetsCompat in = ViewCompat.getRootWindowInsets(decor);
                float d = wv.getResources().getDisplayMetrics().density;
                int[] at = new int[2];
                wv.getLocationInWindow(at);
                int top = 0, bottom = 0, left = 0, right = 0;
                if (in != null) {
                    Insets b = in.getInsets(WindowInsetsCompat.Type.systemBars() | WindowInsetsCompat.Type.displayCutout());
                    top = Math.max(0, b.top - at[1]);
                    bottom = Math.max(0, (at[1] + wv.getHeight()) - (decor.getHeight() - b.bottom));
                    left = Math.max(0, b.left - at[0]);
                    right = Math.max(0, (at[0] + wv.getWidth()) - (decor.getWidth() - b.right));
                }
                r.put("top", Math.round(top / d)); r.put("bottom", Math.round(bottom / d));
                r.put("left", Math.round(left / d)); r.put("right", Math.round(right / d));
            } catch (Exception e) {
                r.put("top", 0); r.put("bottom", 0); r.put("left", 0); r.put("right", 0);
            }
            call.resolve(r);
        });
    }

    /** Leaves the app (as Home does), for the back key on the office's first screen. */
    @PluginMethod
    public void minimize(PluginCall call) {
        getActivity().runOnUiThread(() -> getActivity().moveTaskToBack(true));
        call.resolve();
    }

    @PluginMethod
    public void info(PluginCall call) {
        JSObject r = new JSObject();
        try {
            Context c = getContext();
            r.put("versionCode", PackageInfoCompat.getLongVersionCode(c.getPackageManager().getPackageInfo(c.getPackageName(), 0)));
            r.put("silentUpdate", Build.VERSION.SDK_INT >= 31);
            r.put("canInstall", Build.VERSION.SDK_INT < 26 || c.getPackageManager().canRequestPackageInstalls());
            android.content.SharedPreferences p = c.getSharedPreferences(WatchService.PREFS, Context.MODE_PRIVATE);
            String err = p.getString("update_error", null);
            if (err != null) { r.put("updateError", err); p.edit().remove("update_error").apply(); }
        } catch (Exception e) {
            r.put("versionCode", 0);
        }
        call.resolve(r);
    }
}
