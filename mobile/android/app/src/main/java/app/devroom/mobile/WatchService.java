package app.devroom.mobile;

import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.app.Service;
import android.content.Intent;
import android.content.SharedPreferences;
import android.content.pm.ServiceInfo;
import android.os.Build;
import android.os.IBinder;
import androidx.core.app.NotificationCompat;
import androidx.core.app.NotificationManagerCompat;
import androidx.core.app.ServiceCompat;
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.net.HttpURLConnection;
import java.net.URL;
import java.net.URLEncoder;
import java.util.HashSet;
import java.util.Set;
import org.json.JSONArray;
import org.json.JSONObject;

/**
 * Keeps an eye on the Dev Room app on the user's PC while the phone app is closed,
 * and posts a notification when a session starts waiting for the user.
 */
public class WatchService extends Service {
    static final String PREFS = "devroom";
    private static final String CH_WATCH = "watch";
    private static final String CH_ALERT = "alerts";
    private static final int ONGOING_ID = 1;
    private static final long INTERVAL_MS = 15000;

    private volatile boolean running = false;
    private Thread worker;
    private final Set<String> notified = new HashSet<>();

    @Override
    public void onCreate() {
        super.onCreate();
        if (Build.VERSION.SDK_INT >= 26) {
            NotificationManager nm = getSystemService(NotificationManager.class);
            NotificationChannel watch = new NotificationChannel(CH_WATCH, "Bantay ng Dev Room", NotificationManager.IMPORTANCE_MIN);
            watch.setDescription("Tahimik na paalala na binabantayan ng app ang office mo.");
            NotificationChannel alerts = new NotificationChannel(CH_ALERT, "Naghihintay sa iyo", NotificationManager.IMPORTANCE_HIGH);
            alerts.setDescription("Kapag may session na naghihintay ng sagot mo.");
            nm.createNotificationChannel(watch);
            nm.createNotificationChannel(alerts);
        }
    }

    private PendingIntent openApp(int code) {
        Intent i = new Intent(this, MainActivity.class);
        i.setFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_SINGLE_TOP);
        return PendingIntent.getActivity(this, code, i, PendingIntent.FLAG_UPDATE_CURRENT | PendingIntent.FLAG_IMMUTABLE);
    }

    private android.app.Notification ongoing(String text) {
        return new NotificationCompat.Builder(this, CH_WATCH)
            .setSmallIcon(android.R.drawable.ic_menu_view)
            .setContentTitle("Dev Room")
            .setContentText(text)
            .setOngoing(true)
            .setSilent(true)
            .setContentIntent(openApp(0))
            .build();
    }

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        int type = Build.VERSION.SDK_INT >= 34 ? ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE : 0;
        ServiceCompat.startForeground(this, ONGOING_ID, ongoing("Binabantayan ang office mo."), type);
        // a new pairing starts from scratch: no remembered address or state version
        good = null;
        version = -1;
        if (!running) {
            running = true;
            worker = new Thread(this::loop, "devroom-watch");
            worker.start();
        }
        return START_STICKY;
    }

    // Last address that answered, last state version seen, and how long to wait after failures.
    private String good = null;
    private long version = -1;
    private long backoff = INTERVAL_MS;

    private void loop() {
        loadNotified();
        while (running) {
            try { poll(); backoff = INTERVAL_MS; }
            catch (Exception e) { update("Hindi maabot ang Dev Room sa PC mo. Susubukan ulit."); backoff = Math.min(backoff * 2, 120_000L); }
            try { Thread.sleep(backoff); } catch (InterruptedException e) { return; }
        }
    }

    private void loadNotified() {
        notified.addAll(getSharedPreferences(PREFS, MODE_PRIVATE).getStringSet("notified", new HashSet<>()));
    }

    private void saveNotified() {
        getSharedPreferences(PREFS, MODE_PRIVATE).edit().putStringSet("notified", new HashSet<>(notified)).apply();
    }

    private void update(String text) {
        try { NotificationManagerCompat.from(this).notify(ONGOING_ID, ongoing(text)); } catch (SecurityException ignored) { }
    }

    private HttpURLConnection open(String base, String token) throws Exception {
        URL url = new URL(base.replaceAll("/+$", "") + "/api/state?bg=1&v=" + version + "&t=" + URLEncoder.encode(token, "UTF-8"));
        HttpURLConnection c = (HttpURLConnection) url.openConnection();
        c.setConnectTimeout(4000);
        c.setReadTimeout(8000);
        c.getResponseCode();
        return c;
    }

    private void poll() throws Exception {
        SharedPreferences p = getSharedPreferences(PREFS, MODE_PRIVATE);
        String base = p.getString("url", null);
        if (base == null) { stopSelf(); return; }
        String token = p.getString("token", "");
        String alt = p.getString("alt", "");
        HttpURLConnection c;
        // The address that answered last time first; then home Wi-Fi, then the PC's Tailscale address.
        String first = good != null ? good : base;
        String second = first.equals(base) ? alt : base;
        try { c = open(first, token); good = first; }
        catch (Exception e) { if (second == null || second.isEmpty()) throw e; c = open(second, token); good = second; }
        int code = c.getResponseCode();
        if (code == 204) { c.disconnect(); return; }   // nothing changed since the last poll
        if (code == 401) { c.disconnect(); update("Kailangang i-pair ulit ang phone. I-scan ang QR code sa PC."); running = false; stopSelf(); return; }
        if (code != 200) { c.disconnect(); update("Hindi maabot ang Dev Room sa PC mo. Susubukan ulit."); return; }
        StringBuilder sb = new StringBuilder();
        try (BufferedReader r = new BufferedReader(new InputStreamReader(c.getInputStream(), "UTF-8"))) {
            String line;
            while ((line = r.readLine()) != null) sb.append(line);
        } finally {
            c.disconnect();
        }
        JSONObject j = new JSONObject(sb.toString());
        version = j.optLong("version", -1);
        JSONArray sessions = j.optJSONArray("sessions");
        JSONObject team = j.optJSONObject("team");
        JSONArray ids = team == null ? null : team.optJSONArray("ids");
        Set<String> teamIds = new HashSet<>();
        if (ids != null) for (int i = 0; i < ids.length(); i++) teamIds.add(ids.optString(i));
        Set<String> waitingNow = new HashSet<>();
        int waiting = 0, working = 0;
        if (sessions != null) {
            for (int i = 0; i < sessions.length(); i++) {
                JSONObject s = sessions.getJSONObject(i);
                String id = s.optString("id");
                if (!teamIds.isEmpty() && !teamIds.contains(id)) continue;
                String bucket = s.optString("status_bucket");
                if (bucket.endsWith("WORKING")) working++;
                if (!bucket.endsWith("BLOCKED")) continue;
                waiting++;
                JSONObject pts = s.optJSONObject("post_turn_summary");
                String ask = pts == null ? "" : pts.optString("needs_action");
                // one alert per question, not per status update while it waits
                String key = id + "@" + ask.hashCode();
                waitingNow.add(key);
                if (notified.contains(key)) continue;
                alert(id.hashCode(), s.optString("title", "Isang session"), ask.isEmpty() ? "May tanong siya sa iyo." : ask);
            }
        }
        notified.retainAll(waitingNow);
        notified.addAll(waitingNow);
        saveNotified();
        update(waiting > 0 ? waiting + " ang naghihintay sa iyo. " + working + " ang nagtatrabaho."
            : working > 0 ? working + " ang nagtatrabaho. Walang naghihintay sa iyo." : "Tahimik ang office. Walang naghihintay sa iyo.");
    }

    private void alert(int id, String title, String ask) {
        android.app.Notification n = new NotificationCompat.Builder(this, CH_ALERT)
            .setSmallIcon(android.R.drawable.stat_notify_chat)
            .setContentTitle("Kailangan ka ng " + title)
            .setContentText(ask)
            .setStyle(new NotificationCompat.BigTextStyle().bigText(ask))
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setAutoCancel(true)
            .setContentIntent(openApp(id))
            .build();
        try { NotificationManagerCompat.from(this).notify(1000 + (id & 0xffff), n); } catch (SecurityException ignored) { }
    }

    @Override
    public void onDestroy() {
        running = false;
        if (worker != null) worker.interrupt();
        super.onDestroy();
    }

    @Override
    public IBinder onBind(Intent intent) { return null; }
}
