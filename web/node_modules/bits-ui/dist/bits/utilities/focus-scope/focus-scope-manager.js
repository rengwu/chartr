import { simpleBox } from "svelte-toolbelt";
import { FocusScope } from "./focus-scope.svelte.js";
export class FocusScopeManager {
    static instance;
    #scopeStack = simpleBox([]);
    #focusHistory = new WeakMap();
    #preFocusHistory = new WeakMap();
    static getInstance() {
        if (!this.instance) {
            this.instance = new FocusScopeManager();
        }
        return this.instance;
    }
    register(scope) {
        const current = this.getActive();
        if (current && current !== scope) {
            current.pause();
        }
        // capture the currently focused element before this scope becomes active
        const activeElement = document.activeElement;
        if (activeElement && activeElement !== document.body) {
            this.#preFocusHistory.set(scope, activeElement);
        }
        this.#scopeStack.current = this.#scopeStack.current.filter((s) => s !== scope);
        this.#scopeStack.current.unshift(scope);
    }
    unregister(scope) {
        this.#scopeStack.current = this.#scopeStack.current.filter((s) => s !== scope);
        const next = this.getActive();
        if (next) {
            next.resume();
        }
    }
    getActive() {
        return this.#scopeStack.current[0];
    }
    setFocusMemory(scope, element) {
        this.#focusHistory.set(scope, element);
    }
    getFocusMemory(scope) {
        return this.#focusHistory.get(scope);
    }
    isActiveScope(scope) {
        return this.getActive() === scope;
    }
    setPreFocusMemory(scope, element) {
        this.#preFocusHistory.set(scope, element);
    }
    getPreFocusMemory(scope) {
        return this.#preFocusHistory.get(scope);
    }
    clearPreFocusMemory(scope) {
        this.#preFocusHistory.delete(scope);
    }
}
