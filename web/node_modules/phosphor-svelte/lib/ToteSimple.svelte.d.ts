import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ToteSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <ToteSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ToteSimple: Component<IconComponentProps, {}, "">;
type ToteSimple = ReturnType<typeof ToteSimple>;
export default ToteSimple;
