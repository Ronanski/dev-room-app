package app.devroom.mobile;

import android.annotation.SuppressLint;
import androidx.appcompat.app.AppCompatActivity;
import android.content.Intent;
import android.graphics.Color;
import android.net.Uri;
import android.os.Bundle;
import android.view.Gravity;
import android.view.ViewGroup;
import android.webkit.CookieManager;
import android.webkit.ValueCallback;
import android.webkit.WebChromeClient;
import android.webkit.WebResourceRequest;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.widget.Button;
import android.widget.LinearLayout;

/**
 * The official Claude Code chat (claude.ai/code, with the PC's sessions through Remote Control),
 * shown inside the Dev Room app. Sign-in is kept between visits.
 */
public class ClaudeActivity extends AppCompatActivity {
    private static final int PICK_FILE = 41;
    private WebView web;
    private ValueCallback<Uri[]> pending;

    @SuppressLint("SetJavaScriptEnabled")
    @Override
    protected void onCreate(Bundle state) {
        super.onCreate(state);
        LinearLayout root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setBackgroundColor(Color.parseColor("#1c1613"));

        Button back = new Button(this);
        back.setText("← Dev Room");
        back.setAllCaps(false);
        back.setTextColor(Color.parseColor("#efe6d8"));
        back.setBackgroundColor(Color.parseColor("#2a201a"));
        back.setGravity(Gravity.START | Gravity.CENTER_VERTICAL);
        back.setOnClickListener(v -> finish());
        root.addView(back, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));

        web = new WebView(this);
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
                    try { startActivity(new Intent(Intent.ACTION_VIEW, req.getUrl())); } catch (Exception ignored) { }
                }
                return true;
            }
        });
        web.setWebChromeClient(new WebChromeClient() {
            @Override
            public boolean onShowFileChooser(WebView v, ValueCallback<Uri[]> cb, FileChooserParams p) {
                if (pending != null) pending.onReceiveValue(null);
                pending = cb;
                try { startActivityForResult(p.createIntent(), PICK_FILE); } catch (Exception e) { pending = null; return false; }
                return true;
            }
        });
        root.addView(web, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f));
        setContentView(root);

        String url = getIntent().getStringExtra("url");
        if (url == null || !url.startsWith("https://claude.ai/")) url = "https://claude.ai/code";
        if (state != null) web.restoreState(state); else web.loadUrl(url);
    }

    @Override
    protected void onSaveInstanceState(Bundle out) { super.onSaveInstanceState(out); web.saveState(out); }

    @Override
    protected void onActivityResult(int req, int res, Intent data) {
        super.onActivityResult(req, res, data);
        if (req == PICK_FILE && pending != null) {
            pending.onReceiveValue(WebChromeClient.FileChooserParams.parseResult(res, data));
            pending = null;
        }
    }

    @Override
    public void onBackPressed() {
        if (web.canGoBack()) web.goBack(); else super.onBackPressed();
    }

    @Override
    protected void onPause() { super.onPause(); CookieManager.getInstance().flush(); }
}
