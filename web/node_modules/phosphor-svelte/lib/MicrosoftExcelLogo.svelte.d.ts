import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MicrosoftExcelLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <MicrosoftExcelLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MicrosoftExcelLogo: Component<IconComponentProps, {}, "">;
type MicrosoftExcelLogo = ReturnType<typeof MicrosoftExcelLogo>;
export default MicrosoftExcelLogo;
