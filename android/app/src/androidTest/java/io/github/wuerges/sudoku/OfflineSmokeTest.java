package io.github.wuerges.sudoku;

import android.webkit.WebView;
import androidx.test.ext.junit.runners.AndroidJUnit4;
import androidx.test.platform.app.InstrumentationRegistry;
import androidx.test.rule.ActivityTestRule;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.Rule;
import org.junit.Test;
import org.junit.runner.RunWith;
import static org.junit.Assert.*;

@RunWith(AndroidJUnit4.class)
public class OfflineSmokeTest {
    @Rule public ActivityTestRule<MainActivity> activity = new ActivityTestRule<>(MainActivity.class);
    private String evaluate(String script) throws Exception {
        CountDownLatch done = new CountDownLatch(1);
        AtomicReference<String> value = new AtomicReference<>();
        InstrumentationRegistry.getInstrumentation().runOnMainSync(() -> {
            WebView web = activity.getActivity().getBridge().getWebView();
            web.evaluateJavascript(script, result -> { value.set(result); done.countDown(); });
        });
        assertTrue(done.await(10, TimeUnit.SECONDS));
        return value.get();
    }
    private int navigationId = 0;
    private void navigate(String route) throws Exception {
        String query = "?androidSmoke=" + (++navigationId);
        evaluate("location.href='https://localhost" + route + query + "';");
        // Saved state and a mounted body can belong to the outgoing document.
        // Wait for a unique URL and completed loading in the new document.
        for (int i = 0; i < 120; i++) {
            if ("true".equals(evaluate("location.pathname === '" + route + "' && location.search === '" + query + "' && document.readyState === 'complete' && !!localStorage.getItem('sudoku_state') && !document.getElementById('loading') && document.body.innerText.length > 100"))) return;
            Thread.sleep(500);
        }
        fail("Offline route did not mount: " + route + query);
    }
    @Test public void bundledRoutesAndSaveSurviveReload() throws Exception {
        for (String route : new String[]{"/", "/help", "/config"}) {
            navigate(route);
            assertEquals("Portrait orientation on " + route, android.content.res.Configuration.ORIENTATION_PORTRAIT, activity.getActivity().getResources().getConfiguration().orientation);
            assertEquals("No horizontal overflow on " + route, "true", evaluate("document.documentElement.scrollWidth <= innerWidth"));
            assertEquals("Native worker suppression marker on " + route, "true", evaluate("window.__SUDOKU_NATIVE__ === true"));
            assertEquals("Stable local origin on " + route, "\"https://localhost\"", evaluate("location.origin"));
        }
        String save = evaluate("JSON.parse(localStorage.sudoku_state).board");
        navigate("/config");
        assertEquals("Saved board survives document reload", save, evaluate("JSON.parse(localStorage.sudoku_state).board"));
    }
}
