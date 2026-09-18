import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MicrosoftWordLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <MicrosoftWordLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MicrosoftWordLogo: Component<IconComponentProps, {}, "">;
type MicrosoftWordLogo = ReturnType<typeof MicrosoftWordLogo>;
export default MicrosoftWordLogo;
