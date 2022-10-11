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

import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity

abstract class MillenniumActivity : AppCompatActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
		super.onCreate(savedInstanceState)
		create(this)
	}

	override fun onStart() {
		super.onStart()
		start()
	}

	override fun onResume() {
		super.onResume()
		resume()
	}

	override fun onPause() {
		super.onPause()
		pause()
	}

	override fun onStop() {
		super.onStop()
		stop()
	}

	override fun onWindowFocusChanged(hasFocus: Boolean) {
		super.onWindowFocusChanged(hasFocus)
		focus(hasFocus)
	}

	override fun onSaveInstanceState(outState: Bundle) {
		super.onSaveInstanceState(outState)
		save()
	}

	override fun onDestroy() {
		super.onDestroy()
		destroy()
	}

	override fun onLowMemory() {
		super.onLowMemory()
		memory()
	}

	fun getAppClass(name: String): Class<*> {
		return Class.forName(name)
	}

	companion object {
		init {
			System.loadLibrary("{{app-name-snake-case}}")
		}
	}

	private external fun create(activity: MillenniumActivity)
	private external fun start()
	private external fun resume()
	private external fun pause()
	private external fun stop()
	private external fun save()
	private external fun destroy()
	private external fun memory()
	private external fun focus(focus: Boolean)

	{{class-extension}}
}
