import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AppWindowIcon` instead.
 *
 * @example
 * ```svelte
 * <AppWindow color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AppWindow: Component<IconComponentProps, {}, "">;
type AppWindow = ReturnType<typeof AppWindow>;
export default AppWindow;
