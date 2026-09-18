import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DribbbleLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <DribbbleLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DribbbleLogo: Component<IconComponentProps, {}, "">;
type DribbbleLogo = ReturnType<typeof DribbbleLogo>;
export default DribbbleLogo;
