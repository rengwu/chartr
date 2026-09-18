import type { TooltipTriggerProps } from "../types.js";
declare class __sveltets_Render<T = never> {
    props(): TooltipTriggerProps<T>;
    events(): {};
    slots(): {};
    bindings(): "ref";
    exports(): {};
}
interface $$IsomorphicComponent {
    new <T = never>(options: import('svelte').ComponentConstructorOptions<ReturnType<__sveltets_Render<T>['props']>>): import('svelte').SvelteComponent<ReturnType<__sveltets_Render<T>['props']>, ReturnType<__sveltets_Render<T>['events']>, ReturnType<__sveltets_Render<T>['slots']>> & {
        $$bindings?: ReturnType<__sveltets_Render<T>['bindings']>;
    } & ReturnType<__sveltets_Render<T>['exports']>;
    <T = never>(internal: unknown, props: ReturnType<__sveltets_Render<T>['props']> & {}): ReturnType<__sveltets_Render<T>['exports']>;
    z_$$bindings?: ReturnType<__sveltets_Render<any>['bindings']>;
}
declare const TooltipTrigger: $$IsomorphicComponent;
type TooltipTrigger<T = never> = InstanceType<typeof TooltipTrigger<T>>;
export default TooltipTrigger;
