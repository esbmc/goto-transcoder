;;; goto-transcoder-mode.el --- Defines the elisp functions for this project -*- lexical-binding: t -*-
;;; Commentary:
;;; Code:

(require 'org)
(require 'org-capture)

(defvar goto-transcoder-project-root nil
  "Root directory of the goto-transcoder project.")

(defun goto-transcoder--todo-file ()
  "Return the path to the project TODO.org."
  (when goto-transcoder-project-root
    (expand-file-name "TODO.org" goto-transcoder-project-root)))

(defun goto-transcoder--next-id ()
  "Return the next CUSTOM_ID for a new issue."
  (let ((max-id 0)
        (todo-file (goto-transcoder--todo-file)))
    (when (and todo-file (file-exists-p todo-file))
      (with-current-buffer (find-file-noselect todo-file)
        (org-map-entries
         (lambda ()
           (let ((id (org-entry-get nil "CUSTOM_ID")))
             (when id
               (setq max-id (max max-id (string-to-number id)))))))))
    (number-to-string (1+ max-id))))

(defun goto-transcoder-setup ()
  "Set up the project main commands for use with project."
  (message "GOTO Transcoder Mode Enabled")
  (setq-local compile-command "cargo build")
  (let ((root (locate-dominating-file default-directory "TODO.org")))
    (when root
      (setq goto-transcoder-project-root root)
      (let ((todo-file (expand-file-name "TODO.org" root)))
        (unless (member todo-file org-agenda-files)
          (add-to-list 'org-agenda-files todo-file))
        (let ((template
               `("g" "GOTO Transcoder Issue" entry
                 (file ,todo-file)
                 ,(concat "* TODO [#B] %?\n"
                          ":PROPERTIES:\n"
                          ":CUSTOM_ID: %(goto-transcoder--next-id)\n"
                          ":CREATED: %U\n"
                          ":END:\n")
                 :empty-lines 1)))
          (unless (assoc "g" org-capture-templates)
            (add-to-list 'org-capture-templates template)))))))

(defun goto-transcoder-test ()
  "Run all test cases from the project."
  (interactive)
  (async-shell-command "cargo test -- --ignored"))

(define-minor-mode goto-transcoder-mode
  "A minor mode for the goto-transcoder features."
  :lighter " GotoTranscoder"
  :keymap (let ((map (make-sparse-keymap)))
            (define-key map (kbd "C-x p t") 'goto-transcoder-test)
            map)
  (if goto-transcoder-mode (goto-transcoder-setup)
    ()))

(provide 'goto-transcoder-mode)
;;; goto-transcoder-mode.el ends here
