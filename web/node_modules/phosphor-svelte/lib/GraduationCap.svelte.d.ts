import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GraduationCapIcon` instead.
 *
 * @example
 * ```svelte
 * <GraduationCap color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GraduationCap: Component<IconComponentProps, {}, "">;
type GraduationCap = ReturnType<typeof GraduationCap>;
export default GraduationCap;
