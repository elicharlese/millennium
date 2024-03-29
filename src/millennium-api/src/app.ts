/**
 * Copyright 2022 pyke.io
 *           2019-2021 Tauri Programme within The Commons Conservancy
 *                     [https://tauri.studio/]
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import { invokeMillenniumCommand } from './_internal';

/**
 * Gets the application version defined in the Millennium config file.
 */
export async function getVersion(): Promise<string> {
	return invokeMillenniumCommand<string>({
		__millenniumModule: 'App',
		message: {
			cmd: 'getAppVersion'
		}
	});
}

/**
 * Gets the application name defined in the Millennium config file.
 */
export async function getName(): Promise<string> {
	return invokeMillenniumCommand<string>({
		__millenniumModule: 'App',
		message: {
			cmd: 'getAppName'
		}
	});
}

/**
 * Gets the Millennium version.
 */
export async function getMillenniumVersion(): Promise<string> {
	return invokeMillenniumCommand<string>({
		__millenniumModule: 'App',
		message: {
			cmd: 'getMillenniumVersion'
		}
	});
}

/**
 * Shows the application on macOS. This function does not automatically focus any app window.
 */
export async function show(): Promise<void> {
	return invokeMillenniumCommand({
		__millenniumModule: 'App',
		message: {
			cmd: 'show'
		}
	});
}

/**
 * Hides the application on macOS.
 */
export async function hide(): Promise<void> {
	return invokeMillenniumCommand({
		__millenniumModule: 'App',
		message: {
			cmd: 'hide'
		}
	});
}
