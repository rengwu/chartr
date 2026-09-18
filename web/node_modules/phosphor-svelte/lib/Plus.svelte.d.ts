import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PlusIcon` instead.
 *
 * @example
 * ```svelte
 * <Plus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Plus: Component<IconComponentProps, {}, "">;
type Plus = ReturnType<typeof Plus>;
export default Plus;
