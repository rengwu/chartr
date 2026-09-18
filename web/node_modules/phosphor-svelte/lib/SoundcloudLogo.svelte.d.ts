import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SoundcloudLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <SoundcloudLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SoundcloudLogo: Component<IconComponentProps, {}, "">;
type SoundcloudLogo = ReturnType<typeof SoundcloudLogo>;
export default SoundcloudLogo;
