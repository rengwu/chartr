import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PackageIcon` instead.
 *
 * @example
 * ```svelte
 * <Package color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Package: Component<IconComponentProps, {}, "">;
type Package = ReturnType<typeof Package>;
export default Package;
