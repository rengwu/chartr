import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DotsSixIcon` instead.
 *
 * @example
 * ```svelte
 * <DotsSix color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DotsSix: Component<IconComponentProps, {}, "">;
type DotsSix = ReturnType<typeof DotsSix>;
export default DotsSix;
