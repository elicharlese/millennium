// Copyright 2022 pyke.io
//           2019-2021 Tauri Programme within The Commons Conservancy
//                     [https://tauri.studio/]
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package {{app-domain-reversed}}.{{app-name-snake-case}}

// taken from https://github.com/ionic-team/capacitor/blob/6658bca41e78239347e458175b14ca8bd5c1d6e8/android/capacitor/src/main/java/com/getcapacitor/PermissionHelper.java

import android.content.Context;
import android.content.pm.PackageInfo;
import android.content.pm.PackageManager;
import android.os.Build;
import androidx.core.app.ActivityCompat;
import java.util.ArrayList;
import java.util.Arrays;
import kotlin.collections.List;

object PermissionHelper {
	/**
	 * Checks if a list of given permissions are all granted by the user
	 *
	 * @since 3.0.0
	 * @param permissions Permissions to check.
	 * @return True if all permissions are granted, false if at least one is not.
	 */
	fun hasPermissions(context: Context?, permissions: Array<String>): Boolean {
		for (perm in permissions) {
			if (ActivityCompat.checkSelfPermission(
					context!!,
					perm
				) != PackageManager.PERMISSION_GRANTED
			) {
				return false
			}
		}
		return true
	}

	/**
	 * Check whether the given permission has been defined in the AndroidManifest.xml
	 *
	 * @since 3.0.0
	 * @param permission A permission to check.
	 * @return True if the permission has been defined in the Manifest, false if not.
	 */
	fun hasDefinedPermission(context: Context, permission: String): Boolean {
		var hasPermission = false
		val requestedPermissions = getManifestPermissions(context)
		if (requestedPermissions != null && requestedPermissions.isNotEmpty()) {
			val requestedPermissionsList = listOf(*requestedPermissions)
			val requestedPermissionsArrayList = ArrayList(requestedPermissionsList)
			if (requestedPermissionsArrayList.contains(permission)) {
				hasPermission = true
			}
		}
		return hasPermission
	}

	/**
	 * Check whether all of the given permissions have been defined in the AndroidManifest.xml
	 * @param context the app context
	 * @param permissions a list of permissions
	 * @return true only if all permissions are defined in the AndroidManifest.xml
	 */
	fun hasDefinedPermissions(context: Context, permissions: Array<String>): Boolean {
		for (permission in permissions) {
			if (!hasDefinedPermission(context, permission)) {
				return false
			}
		}
		return true
	}

	/**
	 * Get the permissions defined in AndroidManifest.xml
	 *
	 * @since 3.0.0
	 * @return The permissions defined in AndroidManifest.xml
	 */
	private fun getManifestPermissions(context: Context): Array<String>? {
		var requestedPermissions: Array<String>? = null
		try {
			val pm = context.packageManager
			val packageInfo = if (Build.VERSION.SDK_INT >= 33) {
				pm.getPackageInfo(context.packageName, PackageManager.PackageInfoFlags.of(PackageManager.GET_PERMISSIONS.toLong()))
			} else {
				@Suppress("DEPRECATION")
				pm.getPackageInfo(context.packageName, PackageManager.GET_PERMISSIONS)
			}
			if (packageInfo != null) {
				requestedPermissions = packageInfo.requestedPermissions
			}
		} catch (ex: Exception) {
		}
		return requestedPermissions
	}

	/**
	 * Given a list of permissions, return a new list with the ones not present in AndroidManifest.xml
	 *
	 * @since 3.0.0
	 * @param neededPermissions The permissions needed.
	 * @return The permissions not present in AndroidManifest.xml
	 */
	fun getUndefinedPermissions(context: Context, neededPermissions: Array<String?>): Array<String?> {
		val undefinedPermissions = ArrayList<String?>()
		val requestedPermissions = getManifestPermissions(context)
		if (requestedPermissions != null && requestedPermissions.isNotEmpty()) {
			val requestedPermissionsList = listOf(*requestedPermissions)
			val requestedPermissionsArrayList = ArrayList(requestedPermissionsList)
			for (permission in neededPermissions) {
				if (!requestedPermissionsArrayList.contains(permission)) {
					undefinedPermissions.add(permission)
				}
			}
			var undefinedPermissionArray = arrayOfNulls<String>(undefinedPermissions.size)
			undefinedPermissionArray = undefinedPermissions.toArray(undefinedPermissionArray)
			return undefinedPermissionArray
		}
		return neededPermissions
	}
}
