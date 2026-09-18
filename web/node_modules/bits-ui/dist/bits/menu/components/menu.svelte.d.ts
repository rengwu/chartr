import type { MenuRootProps } from "../types.js";
type $$ComponentProps = MenuRootProps & {
    _internal_variant?: "context-menu" | "dropdown-menu" | "menubar";
    _internal_should_skip_exit_animation?: () => boolean;
};
declare const Menu: import("svelte").Component<$$ComponentProps, {}, "open">;
type Menu = ReturnType<typeof Menu>;
export default Menu;
