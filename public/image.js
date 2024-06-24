/**
 * @typedef {Object} Annotation
 * @property {number} x1
 * @property {number} y1
 * @property {number} x2
 * @property {number} y2
 * @property {boolean} deleted
 */

/** @type HTMLDivElement */
let imageContainerNode = document.querySelector("#container");
/** @type HTMLImageElement */
let imageNode = document.querySelector("#image");
/** @type HTMLButtonElement */
let submitButton = document.querySelector("#submit");

let setSize = () => {
	if (
		imageNode.naturalHeight / imageNode.naturalWidth >
		imageContainerNode.clientHeight / imageContainerNode.clientWidth
	) {
		imageNode.style.height = `${imageContainerNode.clientHeight}px`;
		imageNode.style.width = "auto";
	} else {
		imageNode.style.width = `${imageContainerNode.clientWidth}px`;
		imageNode.style.height = "auto";
	}
};

setSize();
imageNode.addEventListener("load", setSize);
window.addEventListener("resize", setSize);

let submit = () => {
	let scaleFactor = imageNode.naturalWidth / imageNode.clientWidth;

	let newAnnotations = annotations
		.filter((annotation) => !annotation.deleted)
		.map((annotation) => ({
			x1: annotation.x1 * scaleFactor,
			y1: annotation.y1 * scaleFactor,
			x2: annotation.x2 * scaleFactor,
			y2: annotation.y2 * scaleFactor,
		}));

	fetch(`/api/add_annotations/${window.image_id}`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify(newAnnotations),
	})
		.then(() => {
			window.location.replace(window.location.origin);
		})
		.catch(() => {
			console.error("Couldn't add annotations");
		});
};

submitButton.addEventListener("click", (_) => submit());
window.addEventListener("keypress", (e) => {
	if (e.code == "Enter") {
		submit();
	}
});

/**
 * @type Annotation[]
 */
let annotations = [];

imageContainerNode.addEventListener(
	"mousedown",
	(/** @type MouseEvent */ e) => {
		e.preventDefault();
		e.stopPropagation();

		/**
		 * @type Annotation
		 */
		let annotation = {
			x1: e.pageX,
			y1: e.pageY,
			x2: e.pageX,
			y2: e.pageY,
			deleted: false,
		};

		annotations.push(annotation);

		let annotationElement = document.createElement("div");

		annotationElement.className =
			"absolute border-white top-0 left-0 border-2 z-10";

		let tl = document.createElement("div");
		let tr = document.createElement("div");
		let br = document.createElement("div");
		let bl = document.createElement("div");

		let isTooClose = () => {
			return (
				Math.sqrt(
					Math.pow(annotation.x1 - annotation.x2, 2) +
						Math.pow(annotation.y1 - annotation.y2, 2),
				) < 15
			);
		};

		let updatePosition = () => {
			annotationElement.style.transform = `translate(${annotation.x1}px, ${annotation.y1}px)`;
			annotationElement.style.width = `${annotation.x2 - annotation.x1}px`;
			annotationElement.style.height = `${annotation.y2 - annotation.y1}px`;
			if (isTooClose()) {
				tl.classList.add("bg-red-500");
				tr.classList.add("bg-red-500");
				bl.classList.add("bg-red-500");
				br.classList.add("bg-red-500");
			} else {
				tl.classList.remove("bg-red-500");
				tr.classList.remove("bg-red-500");
				bl.classList.remove("bg-red-500");
				br.classList.remove("bg-red-500");
			}
		};

		let listener = (/** @type MouseEvent */ e) => {
			annotation.x2 = e.pageX;
			annotation.y2 = e.pageY;
			updatePosition();
		};

		let upListener = (_) => {
			window.removeEventListener("mousemove", listener);
			window.removeEventListener("mouseup", upListener);
			if (isTooClose()) {
				imageContainerNode.removeChild(annotationElement);
				annotation.deleted = true;
			}
		};

		window.addEventListener("mousemove", listener);
		window.addEventListener("mouseup", upListener);

		updatePosition();

		tl.className =
			"absolute w-4 h-4 rounded-full -translate-x-1/2 -translate-y-1/2 top-0 left-0 bg-gray-900 z-10";

		tl.addEventListener("mousedown", (/** @type MouseEvent */ e) => {
			e.preventDefault();
			e.stopPropagation();

			let listener = (/** @type MouseEvent */ e) => {
				annotation.x1 = Math.min(e.pageX, annotation.x2);
				annotation.y1 = Math.min(e.pageY, annotation.y2);
				updatePosition();
			};

			let upListener = (_) => {
				window.removeEventListener("mousemove", listener);
				window.removeEventListener("mouseup", upListener);
				if (isTooClose()) {
					imageContainerNode.removeChild(annotationElement);
					annotation.deleted = true;
				}
			};

			window.addEventListener("mousemove", listener);
			window.addEventListener("mouseup", upListener);
		});

		annotationElement.appendChild(tl);

		tr.className =
			"absolute w-4 h-4 rounded-full translate-x-1/2 -translate-y-1/2 top-0 right-0 bg-gray-900 z-10";

		tr.addEventListener("mousedown", (/** @type MouseEvent */ e) => {
			e.preventDefault();
			e.stopPropagation();

			let listener = (/** @type MouseEvent */ e) => {
				annotation.x2 = Math.max(e.pageX, annotation.x1);
				annotation.y1 = Math.min(e.pageY, annotation.y2);
				updatePosition();
			};

			let upListener = (_) => {
				window.removeEventListener("mousemove", listener);
				window.removeEventListener("mouseup", upListener);
				if (isTooClose()) {
					imageContainerNode.removeChild(annotationElement);
					annotation.deleted = true;
				}
			};

			window.addEventListener("mousemove", listener);
			window.addEventListener("mouseup", upListener);
		});

		annotationElement.appendChild(tr);

		br.className =
			"absolute w-4 h-4 rounded-full translate-x-1/2 translate-y-1/2 bottom-0 right-0 bg-gray-900 z-10";

		br.addEventListener("mousedown", (/** @type MouseEvent */ e) => {
			e.preventDefault();
			e.stopPropagation();

			let listener = (/** @type MouseEvent */ e) => {
				annotation.x2 = Math.max(e.pageX, annotation.x1);
				annotation.y2 = Math.max(e.pageY, annotation.y1);
				updatePosition();
			};

			let upListener = (_) => {
				window.removeEventListener("mousemove", listener);
				window.removeEventListener("mouseup", upListener);
				if (isTooClose()) {
					imageContainerNode.removeChild(annotationElement);
					annotation.deleted = true;
				}
			};

			window.addEventListener("mousemove", listener);
			window.addEventListener("mouseup", upListener);
		});

		annotationElement.appendChild(br);

		bl.className =
			"absolute w-4 h-4 rounded-full -translate-x-1/2 translate-y-1/2 bottom-0 left-0 bg-gray-900 z-10";

		bl.addEventListener("mousedown", (/** @type MouseEvent */ e) => {
			e.preventDefault();
			e.stopPropagation();

			let listener = (/** @type MouseEvent */ e) => {
				annotation.x1 = Math.min(e.pageX, annotation.x2);
				annotation.y2 = Math.max(e.pageY, annotation.y1);
				updatePosition();
			};

			let upListener = (_) => {
				window.removeEventListener("mousemove", listener);
				window.removeEventListener("mouseup", upListener);
				if (isTooClose()) {
					imageContainerNode.removeChild(annotationElement);
					annotation.deleted = true;
				}
			};

			window.addEventListener("mousemove", listener);
			window.addEventListener("mouseup", upListener);
		});

		annotationElement.appendChild(bl);

		imageContainerNode.appendChild(annotationElement);
	},
);
