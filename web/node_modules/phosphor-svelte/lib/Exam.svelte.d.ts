import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ExamIcon` instead.
 *
 * @example
 * ```svelte
 * <Exam color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Exam: Component<IconComponentProps, {}, "">;
type Exam = ReturnType<typeof Exam>;
export default Exam;
