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

package {{package}}

// taken from https://github.com/ionic-team/capacitor/blob/6658bca41e78239347e458175b14ca8bd5c1d6e8/android/capacitor/src/main/java/com/getcapacitor/Logger.java

import android.text.TextUtils;
import android.util.Log;

class Logger {
	companion object {
		private const val LOG_TAG_CORE = "Millennium"

		fun tags(vararg subtags: String): String {
			return if (subtags.isNotEmpty()) {
				LOG_TAG_CORE + "/" + TextUtils.join("/", subtags)
			} else LOG_TAG_CORE
		}

		fun verbose(message: String) {
			verbose(LOG_TAG_CORE, message)
		}

		private fun verbose(tag: String, message: String) {
			if (!shouldLog()) {
				return
			}
			Log.v(tag, message)
		}

		fun debug(message: String) {
			debug(LOG_TAG_CORE, message)
		}

		fun debug(tag: String, message: String) {
			if (!shouldLog()) {
				return
			}
			Log.d(tag, message)
		}

		fun info(message: String) {
			info(LOG_TAG_CORE, message)
		}

		fun info(tag: String, message: String) {
			if (!shouldLog()) {
				return
			}
			Log.i(tag, message)
		}

		fun warn(message: String) {
			warn(LOG_TAG_CORE, message)
		}

		fun warn(tag: String, message: String) {
			if (!shouldLog()) {
				return
			}
			Log.w(tag, message)
		}

		fun error(message: String) {
			error(LOG_TAG_CORE, message, null)
		}

		fun error(message: String, e: Throwable?) {
			error(LOG_TAG_CORE, message, e)
		}

		fun error(tag: String, message: String, e: Throwable?) {
			if (!shouldLog()) {
				return
			}
			Log.e(tag, message, e)
		}

		private fun shouldLog(): Boolean {
			return BuildConfig.DEBUG
		}
	}
}
