import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MicrosoftPowerpointLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <MicrosoftPowerpointLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MicrosoftPowerpointLogo: Component<IconComponentProps, {}, "">;
type MicrosoftPowerpointLogo = ReturnType<typeof MicrosoftPowerpointLogo>;
export default MicrosoftPowerpointLogo;
