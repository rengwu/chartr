import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UnionIcon` instead.
 *
 * @example
 * ```svelte
 * <Union color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Union: Component<IconComponentProps, {}, "">;
type Union = ReturnType<typeof Union>;
export default Union;
