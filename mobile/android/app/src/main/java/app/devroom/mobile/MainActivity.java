package app.devroom.mobile;

import android.content.Intent;
import android.os.Bundle;
import android.webkit.CookieManager;
import com.getcapacitor.BridgeActivity;

public class MainActivity extends BridgeActivity {
    @Override
    public void onCreate(Bundle savedInstanceState) {
        registerPlugin(DevRoomPlugin.class);
        super.onCreate(savedInstanceState);
    }

    @Override
    protected void onActivityResult(int req, int res, Intent data) {
        if (ClaudeWeb.onResult(req, res, data)) return;
        super.onActivityResult(req, res, data);
    }

    @Override
    public void onPause() { super.onPause(); CookieManager.getInstance().flush(); }
}
