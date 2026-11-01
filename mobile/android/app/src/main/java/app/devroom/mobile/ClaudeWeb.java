package app.devroom.mobile;

import android.annotation.SuppressLint;
import android.app.Activity;
import android.content.Intent;
import android.net.Uri;
import android.webkit.CookieManager;
import android.webkit.ValueCallback;
import android.webkit.WebChromeClient;
import android.webkit.WebResourceRequest;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;

/** The official Claude Code chat (claude.ai/code) as a WebView: shared by the Chat tab overlay and the full Claude screen. */
final class ClaudeWeb {
    static final int PICK_FILE = 41;
    static final String HOME = "https://claude.ai/code";
    private static ValueCallback<Uri[]> pending;

    private ClaudeWeb() { }

    @SuppressLint("SetJavaScriptEnabled")
    static WebView create(Activity act) {
        WebView web = new WebView(act);
        WebSettings s = web.getSettings();
        s.setJavaScriptEnabled(true);
        s.setDomStorageEnabled(true);
        s.setDatabaseEnabled(true);
        s.setMediaPlaybackRequiresUserGesture(false);
        s.setSupportMultipleWindows(false);
        // A plain mobile browser user agent, so sign-in pages treat this like the phone's browser.
        s.setUserAgentString(s.getUserAgentString().replace("; wv", "").replaceAll("Version/[\\d.]+ ", ""));
        CookieManager cm = CookieManager.getInstance();
        cm.setAcceptCookie(true);
        cm.setAcceptThirdPartyCookies(web, true);
        web.setWebViewClient(new WebViewClient() {
            @Override
            public boolean shouldOverrideUrlLoading(WebView view, WebResourceRequest req) {
                String scheme = req.getUrl().getScheme();
                if ("http".equals(scheme) || "https".equals(scheme)) return false;
                // only mail and phone links leave the app; never devroom:, intent: or other app links
                if (("mailto".equals(scheme) || "tel".equals(scheme)) && req.hasGesture()) {
                    try { act.startActivity(new Intent(Intent.ACTION_VIEW, req.getUrl())); } catch (Exception ignored) { }
                }
                return true;
            }
        });
        web.setWebChromeClient(new WebChromeClient() {
            @Override
            public boolean onShowFileChooser(WebView v, ValueCallback<Uri[]> cb, FileChooserParams p) {
                if (pending != null) pending.onReceiveValue(null);
                pending = cb;
                try { act.startActivityForResult(p.createIntent(), PICK_FILE); } catch (Exception e) { pending = null; return false; }
                return true;
            }
        });
        return web;
    }

    /** Call from the hosting activity's onActivityResult. */
    static boolean onResult(int req, int res, Intent data) {
        if (req != PICK_FILE || pending == null) return false;
        pending.onReceiveValue(WebChromeClient.FileChooserParams.parseResult(res, data));
        pending = null;
        return true;
    }
}
