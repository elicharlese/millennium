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

/** File filters for file dialogs. */
export interface FileDialogFilter {
	name: string;

	/**
	 * Extensions to filter, without a `.` prefix.
	 *
	 * @example
	 *
	 * ```typescript
	 * {
	 * 	name: 'Images',
	 * 	extensions: [ 'svg', 'png' ]
	 * }
	 * ```
	 */
	extensions: string[];
}

/** Options for the open dialog. */
export interface OpenDialogOptions {
	/** The title of the dialog window. */
	title?: string;
	/** The file filters of the dialog. */
	filters?: FileDialogFilter[];
	/** Initial directory or file path. */
	defaultPath?: string;
	/** Whether the dialog allows selecting multiple files or not. */
	multiple?: boolean;
	/** Whether the dialog is a directory selection or not. */
	directory?: boolean;
	/**
	 * If `directory` is true, indicates that it will be read recursively later.
	 * Defines whether subdirectories will be allowed on the scope or not.
	 */
	recursive?: boolean;
}

/** Options for the save dialog. */
export interface SaveDialogOptions {
	/** The title of the dialog window. */
	title?: string;
	/** The filters of the dialog. */
	filters?: FileDialogFilter[];
	/**
	 * Initial directory or file path.
	 * If it's a directory path, the dialog interface will change to that folder.
	 * If it's not an existing directory, the file name will be set to the dialog's file name input and the dialog will be set to the parent folder.
	 */
	defaultPath?: string;
}

export interface MessageDialogOptions {
	/** The title of the dialog. Defaults to the app name. */
	title?: string;
	/** The type of the dialog. Defaults to `info`. */
	type?: 'info' | 'warning' | 'error';
}

/**
 * Open a file/directory selection dialog.
 *
 * The selected paths are added to the filesystem and asset protocol allowlist scopes.
 * If security is more important than the ease of use of this API, prefer writing a dedicated command instead.
 *
 * Note that the allowlist scope change is not persisted, so the values are cleared when the application is restarted.
 *
 * @returns A promise resolving to the selected path(s)
 */
export async function open(options: OpenDialogOptions = {}): Promise<null | string | string[]> {
	if (typeof options === 'object')
		Object.freeze(options);

	return await invokeMillenniumCommand({
		__millenniumModule: 'Dialog',
		message: {
			cmd: 'openDialog',
			options
		}
	});
}

/**
 * Open a file/directory save dialog.
 *
 * The selected path is added to the filesystem and asset protocol allowlist scopes.
 * If security is more important than the ease of use of this API, prefer writing a dedicated command instead.
 *
 * Note that the allowlist scope change is not persisted, so the values are cleared when the application is restarted.
 *
 * @returns A promise resolving to the selected path.
 */
export async function save(options: SaveDialogOptions = {}): Promise<string> {
	if (typeof options === 'object')
		Object.freeze(options);

	return await invokeMillenniumCommand({
		__millenniumModule: 'Dialog',
		message: {
			cmd: 'saveDialog',
			options
		}
	});
}

/**
 * Shows a message dialog with an `Ok` button.
 *
 * @example
 * ```typescript
 * import { message } from '@pyke/millennium-api/dialog';
 *
 * await message('This is a dialog', 'Millennium App');
 * await message('File not found', { title: 'Millennium App', type: 'error' });
 * ```
 *
 * @param {string} message The message to show.
 * @param {string | MessageDialogOptions | undefined} options The dialog's options. If a string, it represents the dialog title.
 *
 * @return {Promise<void>} A promise indicating the success or failure of the operation.
 */
export async function message(message: string, options?: string | MessageDialogOptions): Promise<void> {
	const opts = typeof options === 'string' ? { title: options } : options;
	return await invokeMillenniumCommand({
		__millenniumModule: 'Dialog',
		message: {
			cmd: 'messageDialog',
			message: message.toString(),
			title: opts?.title?.toString(),
			type: opts?.type
		}
	});
}

/**
 * Shows a question dialog with `Yes` and `No` buttons.
 *
 * @example
 * ```typescript
 * import { ask } from '@pyke/millennium-api/dialog';
 *
 * const yes = await ask('Are you sure?', 'Millennium App');
 * const yes2 = await ask('This action cannot be reverted. Are you sure?', { title: 'Millennium App', type: 'warning' });
 * ```
 *
 * @param {string} message The message to show.
 * @param {string | MessageDialogOptions | undefined} options The dialog's options. If a string, it represents the dialog title.
 *
 * @return {Promise<void>} A promise resolving to a boolean indicating whether `Yes` was clicked or not.
 */
export async function ask(message: string, options?: string | MessageDialogOptions): Promise<boolean> {
	const opts = typeof options === 'string' ? { title: options } : options;
	return await invokeMillenniumCommand({
		__millenniumModule: 'Dialog',
		message: {
			cmd: 'askDialog',
			message: message.toString(),
			title: opts?.title?.toString(),
			type: opts?.type
		}
	});
}

/**
 * Shows a question dialog with `Ok` and `Cancel` buttons.
 *
 * @example
 * ```typescript
 * import { confirm } from '@pyke/millennium-api/dialog';
 *
 * const confirmed = await confirm('Are you sure?', 'Millennium');
 * const confirmed2 = await confirm('This action cannot be reverted. Are you sure?', { title: 'Millennium', type: 'warning' });
 * ```
 *
 * @param {string} message The message to show.
 * @param {string | MessageDialogOptions | undefined} options The dialog's options. If a string, it represents the dialog title.
 *
 * @return {Promise<void>} A promise resolving to a boolean indicating whether `Ok` was clicked or not.
 */
export async function confirm(message: string, options?: string | MessageDialogOptions): Promise<boolean> {
	const opts = typeof options === 'string' ? { title: options } : options;
	return await invokeMillenniumCommand({
		__millenniumModule: 'Dialog',
		message: {
			cmd: 'confirmDialog',
			message: message.toString(),
			title: opts?.title?.toString(),
			type: opts?.type
		}
	});
}
