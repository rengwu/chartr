import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PencilSimpleLineIcon` instead.
 *
 * @example
 * ```svelte
 * <PencilSimpleLine color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PencilSimpleLine: Component<IconComponentProps, {}, "">;
type PencilSimpleLine = ReturnType<typeof PencilSimpleLine>;
export default PencilSimpleLine;
