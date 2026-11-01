package app.devroom.mobile;

import androidx.appcompat.app.AppCompatActivity;
import android.content.Intent;
import android.graphics.Color;
import android.os.Bundle;
import android.view.Gravity;
import android.view.ViewGroup;
import android.webkit.CookieManager;
import android.webkit.WebView;
import android.widget.Button;
import android.widget.LinearLayout;

/**
 * The official Claude Code chat as a full screen, for office pages that cannot place it on the Chat tab.
 * Sign-in is kept between visits (shared cookies with the Chat tab view).
 */
public class ClaudeActivity extends AppCompatActivity {
    private WebView web;

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

        web = ClaudeWeb.create(this);
        root.addView(web, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f));
        setContentView(root);
        // edge to edge on new Android: keep the bar below the status bar and the chat above the keyboard
        androidx.core.view.ViewCompat.setOnApplyWindowInsetsListener(root, (v, in) -> {
            androidx.core.graphics.Insets i = in.getInsets(androidx.core.view.WindowInsetsCompat.Type.systemBars() | androidx.core.view.WindowInsetsCompat.Type.ime());
            v.setPadding(i.left, i.top, i.right, i.bottom);
            return androidx.core.view.WindowInsetsCompat.CONSUMED;
        });
        getOnBackPressedDispatcher().addCallback(this, new androidx.activity.OnBackPressedCallback(true) {
            @Override public void handleOnBackPressed() { if (web.canGoBack()) web.goBack(); else finish(); }
        });

        String url = getIntent().getStringExtra("url");
        if (url == null || !url.startsWith("https://claude.ai/")) url = ClaudeWeb.HOME;
        if (state != null) web.restoreState(state); else web.loadUrl(url);
    }

    @Override
    protected void onSaveInstanceState(Bundle out) { super.onSaveInstanceState(out); web.saveState(out); }

    @Override
    protected void onActivityResult(int req, int res, Intent data) {
        super.onActivityResult(req, res, data);
        ClaudeWeb.onResult(req, res, data);
    }

    @Override
    protected void onPause() { super.onPause(); CookieManager.getInstance().flush(); }
}
