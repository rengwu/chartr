import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StudentIcon` instead.
 *
 * @example
 * ```svelte
 * <Student color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Student: Component<IconComponentProps, {}, "">;
type Student = ReturnType<typeof Student>;
export default Student;
