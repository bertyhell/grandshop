import { applyMiddleware, combineReducers, createStore, Reducer } from 'redux';
import { composeWithDevTools } from 'redux-devtools-extension';
import thunk from 'redux-thunk';

import appReducer from './reducer';
import { AppState } from './types';

const middleware = [thunk];

export default createStore(
    combineReducers<Reducer<AppState>>({
      app: appReducer
    }),
    composeWithDevTools(applyMiddleware(...middleware))
);
