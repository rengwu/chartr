import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PencilRulerIcon` instead.
 *
 * @example
 * ```svelte
 * <PencilRuler color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PencilRuler: Component<IconComponentProps, {}, "">;
type PencilRuler = ReturnType<typeof PencilRuler>;
export default PencilRuler;
