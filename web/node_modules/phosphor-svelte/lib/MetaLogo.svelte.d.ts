import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MetaLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <MetaLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MetaLogo: Component<IconComponentProps, {}, "">;
type MetaLogo = ReturnType<typeof MetaLogo>;
export default MetaLogo;
