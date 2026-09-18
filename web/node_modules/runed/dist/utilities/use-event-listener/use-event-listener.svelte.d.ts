import type { MaybeGetter } from "../../internal/types.js";
export type EventHandler<TEvent extends Event, TTarget extends EventTarget> = (this: TTarget, event: TEvent & {
    currentTarget: TTarget;
}) => unknown;
/**
 * Adds an event listener to the specified target element for the given event(s), and returns a function to remove it.
 * @param target The target element to add the event listener to.
 * @param event The event(s) to listen for.
 * @param handler The function to be called when the event is triggered.
 * @param options An optional object that specifies characteristics about the event listener.
 *
 * @see {@link https://runed.dev/docs/utilities/use-event-listener}
 */
export declare function useEventListener<TEvent extends keyof WindowEventMap>(target: MaybeGetter<Window | null | undefined>, event: MaybeGetter<TEvent | TEvent[]>, handler: EventHandler<WindowEventMap[TEvent], Window>, options?: AddEventListenerOptions): void;
/**
 * Adds an event listener to the specified target element for the given event(s), and returns a function to remove it.
 * @param target The target element to add the event listener to.
 * @param event The event(s) to listen for.
 * @param handler The function to be called when the event is triggered.
 * @param options An optional object that specifies characteristics about the event listener.
 *
 * @see {@link https://runed.dev/docs/utilities/use-event-listener}
 */
export declare function useEventListener<TEvent extends keyof DocumentEventMap>(target: MaybeGetter<Document | null | undefined>, event: MaybeGetter<TEvent | TEvent[]>, handler: EventHandler<DocumentEventMap[TEvent], Document>, options?: AddEventListenerOptions): void;
/**
 * Adds an event listener to the specified target element for the given event(s), and returns a function to remove it.
 * @param target The target element to add the event listener to.
 * @param event The event(s) to listen for.
 * @param handler The function to be called when the event is triggered.
 * @param options An optional object that specifies characteristics about the event listener.
 *
 * @see {@link https://runed.dev/docs/utilities/use-event-listener}
 */
export declare function useEventListener<TElement extends HTMLElement, TEvent extends keyof HTMLElementEventMap>(target: MaybeGetter<TElement | null | undefined>, event: MaybeGetter<TEvent | TEvent[]>, handler: EventHandler<HTMLElementEventMap[TEvent], TElement>, options?: AddEventListenerOptions): void;
/**
 * Adds an event listener to the specified target element for the given event(s), and returns a function to remove it.
 * @param target The target element to add the event listener to.
 * @param event The event(s) to listen for.
 * @param handler The function to be called when the event is triggered.
 * @param options An optional object that specifies characteristics about the event listener.
 *
 * @see {@link https://runed.dev/docs/utilities/use-event-listener}
 */
export declare function useEventListener<TEvent extends keyof MediaQueryListEventMap>(target: MaybeGetter<MediaQueryList | null | undefined>, event: MaybeGetter<TEvent | TEvent[]>, handler: EventHandler<MediaQueryListEventMap[TEvent], MediaQueryList>, options?: AddEventListenerOptions): void;
/**
 * Adds an event listener to the specified target element for the given event(s), and returns a function to remove it.
 * @param target The target element to add the event listener to.
 * @param event The event(s) to listen for.
 * @param handler The function to be called when the event is triggered.
 * @param options An optional object that specifies characteristics about the event listener.
 *
 * @see {@link https://runed.dev/docs/utilities/use-event-listener}
 */
export declare function useEventListener(target: MaybeGetter<EventTarget | null | undefined>, event: MaybeGetter<string | string[]>, handler: EventListener, options?: AddEventListenerOptions): void;
