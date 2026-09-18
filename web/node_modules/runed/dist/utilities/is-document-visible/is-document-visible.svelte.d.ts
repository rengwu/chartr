import { type ConfigurableDocument, type ConfigurableWindow } from "../../internal/configurable-globals.js";
export type IsDocumentVisibleOptions = ConfigurableWindow & ConfigurableDocument;
/**
 * Tracks whether the current document is visible (i.e., not hidden).
 * It listens to the "visibilitychange" event and updates reactively.
 *
 * @see {@link https://developer.mozilla.org/docs/Web/API/Document/visibilitychange_event}
 * @see {@link https://runed.dev/docs/utilities/is-document-visible}
 */
export declare class IsDocumentVisible {
    #private;
    constructor(options?: IsDocumentVisibleOptions);
    get current(): boolean;
}
