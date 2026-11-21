package app.loglens.mobile

import android.Manifest
import android.content.pm.PackageManager
import android.graphics.Color
import android.os.Build
import android.os.Bundle
import androidx.activity.SystemBarStyle
import androidx.activity.enableEdgeToEdge

private const val LOCAL_NETWORK_REQUEST = 1
private const val LOCAL_NETWORK_MIN_SDK = 37

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge(
      statusBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
      navigationBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
    )
    super.onCreate(savedInstanceState)
    requestLocalNetworkAccess()
  }

  // Android 17+ blocks LAN and emulator-host addresses (the LogLens server) without this runtime permission.
  private fun requestLocalNetworkAccess() {
    if (Build.VERSION.SDK_INT < LOCAL_NETWORK_MIN_SDK) return
    val permission = Manifest.permission.ACCESS_LOCAL_NETWORK
    if (checkSelfPermission(permission) != PackageManager.PERMISSION_GRANTED) {
      requestPermissions(arrayOf(permission), LOCAL_NETWORK_REQUEST)
    }
  }
}
