package app.devroom.mobile;

import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.pm.PackageInstaller;
import android.os.Build;
import androidx.core.app.NotificationCompat;
import androidx.core.content.ContextCompat;

/**
 * Self-updates (0.17.0).
 * - The result of an install session: on Android 12+ a self-update installs without a tap. If Android still
 *   wants a confirmation, it is opened right away, and a notification ("I-tap para i-install") is posted too,
 *   since a screen cannot be opened from the background.
 * - After the app was replaced: the background watcher (alerts) is started again, so alerts keep coming
 *   without opening the app.
 */
public class UpdateReceiver extends BroadcastReceiver {
    static final String ACTION_RESULT = "app.devroom.mobile.UPDATE_RESULT";
    private static final String CH = "devroom-update";
    private static final int NOTE_ID = 4711;

    @Override
    public void onReceive(Context ctx, Intent intent) {
        String a = intent.getAction();
        if (Intent.ACTION_MY_PACKAGE_REPLACED.equals(a)) {
            ctx.getSharedPreferences(WatchService.PREFS, Context.MODE_PRIVATE).edit().remove("update_error").apply();
            String url = ctx.getSharedPreferences(WatchService.PREFS, Context.MODE_PRIVATE).getString("url", "");
            if (url != null && !url.isEmpty()) {
                try { ContextCompat.startForegroundService(ctx, new Intent(ctx, WatchService.class)); } catch (Exception ignored) { }
            }
            return;
        }
        if (!ACTION_RESULT.equals(a)) return;
        int status = intent.getIntExtra(PackageInstaller.EXTRA_STATUS, PackageInstaller.STATUS_FAILURE);
        if (status == PackageInstaller.STATUS_PENDING_USER_ACTION) {
            Intent confirm = Build.VERSION.SDK_INT >= 33
                ? intent.getParcelableExtra(Intent.EXTRA_INTENT, Intent.class)
                : intent.getParcelableExtra(Intent.EXTRA_INTENT);
            if (confirm == null) return;
            confirm.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
            try { ctx.startActivity(confirm); } catch (Exception ignored) { }
            notifyConfirm(ctx, confirm);
        } else if (status != PackageInstaller.STATUS_SUCCESS) {
            String msg = intent.getStringExtra(PackageInstaller.EXTRA_STATUS_MESSAGE);
            ctx.getSharedPreferences(WatchService.PREFS, Context.MODE_PRIVATE).edit()
                .putString("update_error", "Hindi na-install ang update" + (msg != null ? ": " + msg : ".")).apply();
        }
    }

    private static void notifyConfirm(Context ctx, Intent confirm) {
        NotificationManager nm = (NotificationManager) ctx.getSystemService(Context.NOTIFICATION_SERVICE);
        if (nm == null) return;
        if (Build.VERSION.SDK_INT >= 26) nm.createNotificationChannel(new NotificationChannel(CH, "Mga update ng app", NotificationManager.IMPORTANCE_DEFAULT));
        PendingIntent pi = PendingIntent.getActivity(ctx, NOTE_ID, confirm, PendingIntent.FLAG_UPDATE_CURRENT | PendingIntent.FLAG_IMMUTABLE);
        nm.notify(NOTE_ID, new NotificationCompat.Builder(ctx, CH)
            .setSmallIcon(android.R.drawable.stat_sys_download_done)
            .setContentTitle("May bagong Dev Room app")
            .setContentText("I-tap para i-install. Mananatili ang pairing mo.")
            .setContentIntent(pi)
            .setAutoCancel(true)
            .build());
    }
}
