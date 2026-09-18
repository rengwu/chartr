export type EventCallback<E extends Event = Event> = (event: E) => void;
/**
 * Creates a typed event dispatcher and listener pair for custom events
 * @template T - The type of data that will be passed in the event detail
 * @param eventName - The name of the custom event
 * @param options - CustomEvent options (bubbles, cancelable, etc.)
 */
export declare class CustomEventDispatcher<T = unknown> {
    readonly eventName: string;
    readonly options: Omit<CustomEventInit<T>, "detail">;
    constructor(eventName: string, options?: Omit<CustomEventInit<T>, "detail">);
    createEvent(detail?: T): CustomEvent<T>;
    dispatch(element: EventTarget, detail?: T): CustomEvent<T>;
    listen(element: EventTarget, callback: (event: CustomEvent<T>) => void, options?: AddEventListenerOptions): () => void;
}
