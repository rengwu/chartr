import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SlackLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <SlackLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SlackLogo: Component<IconComponentProps, {}, "">;
type SlackLogo = ReturnType<typeof SlackLogo>;
export default SlackLogo;
