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

import android.annotation.SuppressLint
import android.webkit.*
import android.content.Context
import android.os.Build
import kotlin.collections.Map

class RustWebView(context: Context): WebView(context) {
    init {
        settings.javaScriptEnabled = true
		settings.domStorageEnabled = true
		settings.setGeolocationEnabled(true)
		settings.databaseEnabled = true
		settings.mediaPlaybackRequiresUserGesture = false
		settings.javaScriptCanOpenWindowsAutomatically = true
        {{class-init}}
    }

	fun loadUrlMainThread(url: String) {
		post {
			super.loadUrl(url)
		}
	}

    fun loadUrlMainThread(url: String, additionalHttpHeaders: Map<String, String>) {
        post {
          super.loadUrl(url, additionalHttpHeaders)
        }
    }

    {{class-extension}}
}
