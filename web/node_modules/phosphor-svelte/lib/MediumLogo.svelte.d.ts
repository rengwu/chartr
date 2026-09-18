import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MediumLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <MediumLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MediumLogo: Component<IconComponentProps, {}, "">;
type MediumLogo = ReturnType<typeof MediumLogo>;
export default MediumLogo;
