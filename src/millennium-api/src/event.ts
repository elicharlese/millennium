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

import type { WindowLabel } from './window';
import { invokeMillenniumCommand } from './_internal';
import { transformCallback } from './millennium';

import type { LiteralUnion } from 'type-fest';

export interface Event<T> {
	event: EventName;
	/** The label of the window that emitted this event. */
	windowLabel: string;
	/** The unique identifier for this event. */
	id: number;
	payload: T;
}

export const enum MillenniumEvent {
	WINDOW_RESIZED = 'millennium://resize',
	WINDOW_MOVED = 'millennium://move',
	WINDOW_CLOSE_REQUESTED = 'millennium://close-requested',
	WINDOW_CREATED = 'millennium://window-created',
	WINDOW_DESTROYED = 'millennium://window-destroyed',
	WINDOW_FOCUS = 'millennium://focus',
	WINDOW_BLUR = 'millennium://blur',
	WINDOW_SCALE_FACTOR_CHANGED = 'millennium://scale-change',
	WINDOW_THEME_CHANGED = 'millennium://theme-changed',
	WINDOW_FILE_DROP = 'millennium://file-drop',
	WINDOW_FILE_DROP_HOVER = 'millennium://file-drop-hover',
	WINDOW_FILE_DROP_CANCELLED = 'millennium://file-drop-cancelled',
	MENU = 'millennium://menu',
	UPDATE_CHECK = 'millennium://update',
	UPDATE_AVAILABLE = 'millennium://update-available',
	UPDATE_INSTALL = 'millennium://update-install',
	UPDATE_STATUS = 'millennium://update-status',
	UPDATE_DOWNLOAD_PROGRESS = 'millennium://update-download-progress',
	ERROR = 'millennium://error'
}

export type EventName = LiteralUnion<`${MillenniumEvent}`, string>;

export type EventCallback<T> = (event: Event<T>) => void;

export type Unlistener = () => void;

/**
 * Unregister the event listener associated with the given event name and ID.
 */
async function unlisten(event: string, eventId: number): Promise<void> {
	return invokeMillenniumCommand({
		__millenniumModule: 'Event',
		message: {
			cmd: 'unlisten',
			event,
			eventId
		}
	});
}

/**
 * Emits an event to the backend.
 *
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:`, and `_`.
 * @param windowLabel The label of the window to which the event is sent. If null, the event is sent to all windows.
 */
export async function emit(event: EventName | string, windowLabel?: WindowLabel | null, payload?: unknown): Promise<void> {
	await invokeMillenniumCommand({
		__millenniumModule: 'Event',
		message: {
			cmd: 'emit',
			event,
			windowLabel,
			payload
		}
	});
}

/**
 * Listen to an event from the backend.
 *
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:`, and `_`.
 */
export async function listen<T>(event: EventName, windowLabel: string | null, handler: EventCallback<T>): Promise<Unlistener> {
	const eventId = await invokeMillenniumCommand<number>({
		__millenniumModule: 'Event',
		message: {
			cmd: 'listen',
			event,
			windowLabel,
			handler: transformCallback(handler)
		}
	});
	return async () => unlisten(event, eventId);
}

/**
 * Listen to a one-off event from the backend.
 *
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:`, and `_`.
 */
export async function once<T>(event: EventName, windowLabel: string | null, handler: EventCallback<T>): Promise<Unlistener> {
	return listen<T>(event, windowLabel, (eventData) => {
		handler(eventData);
		unlisten(event, eventData.id).catch(() => {});
	});
}
