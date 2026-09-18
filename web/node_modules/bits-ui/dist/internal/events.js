import { on } from "svelte/events";
/**
 * Creates a typed event dispatcher and listener pair for custom events
 * @template T - The type of data that will be passed in the event detail
 * @param eventName - The name of the custom event
 * @param options - CustomEvent options (bubbles, cancelable, etc.)
 */
export class CustomEventDispatcher {
    eventName;
    options;
    constructor(eventName, options = { bubbles: true, cancelable: true }) {
        this.eventName = eventName;
        this.options = options;
    }
    createEvent(detail) {
        return new CustomEvent(this.eventName, {
            ...this.options,
            detail,
        });
    }
    dispatch(element, detail) {
        const event = this.createEvent(detail);
        element.dispatchEvent(event);
        return event;
    }
    listen(element, callback, options) {
        const handler = (event) => {
            callback(event);
        };
        return on(element, this.eventName, handler, options);
    }
}
