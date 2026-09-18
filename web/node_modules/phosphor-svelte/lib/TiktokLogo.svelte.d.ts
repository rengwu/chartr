import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TiktokLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <TiktokLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TiktokLogo: Component<IconComponentProps, {}, "">;
type TiktokLogo = ReturnType<typeof TiktokLogo>;
export default TiktokLogo;
