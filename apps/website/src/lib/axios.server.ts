import axios from 'axios';
import { API_IP, API_PORT } from '$env/static/private';

export const api = axios.create({
	baseURL: `http://${API_IP}:${API_PORT}/`
});
