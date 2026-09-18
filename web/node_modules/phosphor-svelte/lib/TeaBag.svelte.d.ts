import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TeaBagIcon` instead.
 *
 * @example
 * ```svelte
 * <TeaBag color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TeaBag: Component<IconComponentProps, {}, "">;
type TeaBag = ReturnType<typeof TeaBag>;
export default TeaBag;
