import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <MicrosoftWordLogoIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MicrosoftWordLogoIcon: Component<IconComponentProps, {}, "">;
type MicrosoftWordLogoIcon = ReturnType<typeof MicrosoftWordLogoIcon>;
export default MicrosoftWordLogoIcon;
