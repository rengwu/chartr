import type { TooltipRootProps } from "../types.js";
declare class __sveltets_Render<T = never> {
    props(): TooltipRootProps<T>;
    events(): {};
    slots(): {};
    bindings(): "open" | "triggerId";
    exports(): {};
}
interface $$IsomorphicComponent {
    new <T = never>(options: import('svelte').ComponentConstructorOptions<ReturnType<__sveltets_Render<T>['props']>>): import('svelte').SvelteComponent<ReturnType<__sveltets_Render<T>['props']>, ReturnType<__sveltets_Render<T>['events']>, ReturnType<__sveltets_Render<T>['slots']>> & {
        $$bindings?: ReturnType<__sveltets_Render<T>['bindings']>;
    } & ReturnType<__sveltets_Render<T>['exports']>;
    <T = never>(internal: unknown, props: ReturnType<__sveltets_Render<T>['props']> & {}): ReturnType<__sveltets_Render<T>['exports']>;
    z_$$bindings?: ReturnType<__sveltets_Render<any>['bindings']>;
}
declare const Tooltip: $$IsomorphicComponent;
type Tooltip<T = never> = InstanceType<typeof Tooltip<T>>;
export default Tooltip;
