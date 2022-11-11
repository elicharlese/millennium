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

import { once, listen, emit, Unlistener, MillenniumEvent } from './event';

export type UpdateStatus = 'PENDING' | 'ERROR' | 'DONE' | 'UPTODATE';

interface UpdateStatusResult {
	error?: string;
	status: UpdateStatus;
}

export interface UpdateManifest {
	version: string;
	date: string;
	body: string;
}

export interface UpdateResult {
	manifest?: UpdateManifest;
	shouldUpdate: boolean;
}

/**
 * Listen to an updater event.
 * @example
 * ```typescript
 * import { onUpdaterEvent } from '@pyke/millennium-api/updater';
 * const unlisten = await onUpdaterEvent(({ error, status }) => {
 * 	console.log('Updater event', error, status);
 * });
 *
 * // you need to call unlisten if your handler goes out of scope, e.g. if a component is unmounted
 * unlisten();
 * ```
 *
 * @param handler
 * @returns A promise resolving to a function to unlisten to the event.
 * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
 */
export async function onUpdaterEvent(handler: (status: UpdateStatusResult) => void): Promise<Unlistener> {
	return listen(MillenniumEvent.UPDATE_STATUS, null, (data: { payload: any }) => {
		handler(data?.payload as UpdateStatusResult);
	});
}

export function installUpdate(): Promise<void> {
	let unlistenerFn: Unlistener | undefined;

	function cleanListener(): void {
		unlistenerFn?.();
		unlistenerFn = undefined;
	}

	return new Promise((resolve, reject) => {
		function onStatusChange(statusResult: UpdateStatusResult): void {
			if (statusResult.error) {
				cleanListener();
				return reject(statusResult.error);
			}

			if (statusResult.status === 'DONE') {
				cleanListener();
				return resolve();
			}
		}

		onUpdaterEvent(onStatusChange)
			.then(fn => {
				unlistenerFn = fn;
			})
			.catch(e => {
				cleanListener();
				throw e;
			});

		emit(MillenniumEvent.UPDATE_INSTALL).catch(e => {
			cleanListener();
			throw e;
		});
	});
}

export function checkForUpdates(): Promise<UpdateResult> {
	let unlistenerFn: Unlistener | undefined;

	function cleanListener(): void {
		unlistenerFn?.();
		unlistenerFn = undefined;
	}

	return new Promise((resolve, reject) => {
		function onUpdateAvailable(manifest: UpdateManifest): void {
			cleanListener();
			return resolve({
				manifest,
				shouldUpdate: true
			});
		}

		function onStatusChange(statusResult: UpdateStatusResult): void {
			if (statusResult.error) {
				cleanListener();
				return reject(statusResult.error);
			}

			if (statusResult.status === 'UPTODATE') {
				cleanListener();
				return resolve({
					shouldUpdate: false
				});
			}
		}

		once(MillenniumEvent.UPDATE_AVAILABLE, null, (data: { payload: any }) => {
			onUpdateAvailable(data?.payload as UpdateManifest);
		}).catch(e => {
			cleanListener();
			throw e;
		});

		onUpdaterEvent(onStatusChange)
			.then(fn => {
				unlistenerFn = fn;
			})
			.catch(e => {
				cleanListener();
				throw e;
			});

		emit(MillenniumEvent.UPDATE_CHECK).catch(e => {
			cleanListener();
			throw e;
		});
	});
}
