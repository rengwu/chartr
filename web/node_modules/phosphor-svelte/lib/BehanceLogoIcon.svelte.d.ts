import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <BehanceLogoIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BehanceLogoIcon: Component<IconComponentProps, {}, "">;
type BehanceLogoIcon = ReturnType<typeof BehanceLogoIcon>;
export default BehanceLogoIcon;
