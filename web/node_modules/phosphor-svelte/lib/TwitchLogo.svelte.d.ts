import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TwitchLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <TwitchLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TwitchLogo: Component<IconComponentProps, {}, "">;
type TwitchLogo = ReturnType<typeof TwitchLogo>;
export default TwitchLogo;
