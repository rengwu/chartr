import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ListNumbersIcon` instead.
 *
 * @example
 * ```svelte
 * <ListNumbers color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ListNumbers: Component<IconComponentProps, {}, "">;
type ListNumbers = ReturnType<typeof ListNumbers>;
export default ListNumbers;
