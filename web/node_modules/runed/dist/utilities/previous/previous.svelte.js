/**
 * Holds the previous value of a getter.
 *
 * @see {@link https://runed.dev/docs/utilities/previous}
 */
export class Previous {
    #previousCallback = () => undefined;
    #previous = $derived.by(() => this.#previousCallback());
    constructor(getter, initialValue) {
        let actualPrevious = undefined;
        if (initialValue !== undefined)
            actualPrevious = initialValue;
        this.#previousCallback = () => {
            try {
                return actualPrevious;
            }
            finally {
                actualPrevious = getter();
            }
        };
    }
    get current() {
        return this.#previous;
    }
}
