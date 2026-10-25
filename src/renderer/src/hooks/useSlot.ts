import { ReactElement, ReactNode } from 'react';

export function useSlot<T>(type: T, children: ReactNode): ReactNode {
	const slots: ReactNode[] = Array.isArray(children) ? children : [children];
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const element = slots.find((c: any) => c && c.type === type) as
		ReactElement<{ children?: ReactNode }> | undefined;

	if (element && element.props.children) {
		return element.props.children;
	}

	return null;
}
