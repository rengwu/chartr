import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ToiletPaperIcon` instead.
 *
 * @example
 * ```svelte
 * <ToiletPaper color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ToiletPaper: Component<IconComponentProps, {}, "">;
type ToiletPaper = ReturnType<typeof ToiletPaper>;
export default ToiletPaper;
