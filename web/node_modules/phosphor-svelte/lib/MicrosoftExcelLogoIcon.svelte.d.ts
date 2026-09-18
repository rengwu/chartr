import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <MicrosoftExcelLogoIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MicrosoftExcelLogoIcon: Component<IconComponentProps, {}, "">;
type MicrosoftExcelLogoIcon = ReturnType<typeof MicrosoftExcelLogoIcon>;
export default MicrosoftExcelLogoIcon;
