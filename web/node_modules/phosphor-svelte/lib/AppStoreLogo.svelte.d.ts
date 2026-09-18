import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AppStoreLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <AppStoreLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AppStoreLogo: Component<IconComponentProps, {}, "">;
type AppStoreLogo = ReturnType<typeof AppStoreLogo>;
export default AppStoreLogo;
