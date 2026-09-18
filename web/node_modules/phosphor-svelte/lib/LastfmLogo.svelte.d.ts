import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LastfmLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <LastfmLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LastfmLogo: Component<IconComponentProps, {}, "">;
type LastfmLogo = ReturnType<typeof LastfmLogo>;
export default LastfmLogo;
