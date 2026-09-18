import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PaperPlaneRightIcon` instead.
 *
 * @example
 * ```svelte
 * <PaperPlaneRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PaperPlaneRight: Component<IconComponentProps, {}, "">;
type PaperPlaneRight = ReturnType<typeof PaperPlaneRight>;
export default PaperPlaneRight;
