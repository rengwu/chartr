import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NotionLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <NotionLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NotionLogo: Component<IconComponentProps, {}, "">;
type NotionLogo = ReturnType<typeof NotionLogo>;
export default NotionLogo;
