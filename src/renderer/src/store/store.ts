import { configureStore } from '@reduxjs/toolkit';

import appReducer from './reducer';

const store = configureStore({
	reducer: {
		app: appReducer,
	},
	middleware: (getDefaultMiddleware) =>
		// Image buffers are stored as base64 strings, which makes the dev checks very slow
		getDefaultMiddleware({ immutableCheck: false, serializableCheck: false }),
});

export type AppDispatch = typeof store.dispatch;

export default store;
